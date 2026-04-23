import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'

const VERSION_TAG_PATTERN = /^v(\d+)\.(\d+)\.(\d+)$/
const ZERO_SHA = /^0+$/
const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')

export function parseVersionTag(tag) {
  const match = VERSION_TAG_PATTERN.exec(tag)
  if (!match) {
    return null
  }

  return {
    major: Number(match[1]),
    minor: Number(match[2]),
    patch: Number(match[3]),
  }
}

export function formatVersion({ major, minor, patch }) {
  return `${major}.${minor}.${patch}`
}

export function determineBumpType(totalChangedLines, thresholds = {}) {
  const patchMaxLines = Number(thresholds.patchMaxLines ?? 50)
  const minorMaxLines = Number(thresholds.minorMaxLines ?? 200)

  if (totalChangedLines > minorMaxLines) {
    return 'major'
  }

  if (totalChangedLines > patchMaxLines) {
    return 'minor'
  }

  return 'patch'
}

export function extractChangedLineCount(summary) {
  const insertions = summary.match(/(\d+) insertions?\(\+\)/)
  const deletions = summary.match(/(\d+) deletions?\(-\)/)

  return Number(insertions?.[1] ?? 0) + Number(deletions?.[1] ?? 0)
}

function ensureGlobalFlags(flags) {
  const normalized = new Set(flags.split(''))
  normalized.add('g')
  return [...normalized].join('')
}

export function incrementVersion(version, bumpType) {
  switch (bumpType) {
    case 'major':
      return { major: version.major + 1, minor: 0, patch: 0 }
    case 'minor':
      return { major: version.major, minor: version.minor + 1, patch: 0 }
    case 'patch':
      return { major: version.major, minor: version.minor, patch: version.patch + 1 }
    default:
      throw new Error(`Unsupported bump type: ${bumpType}`)
  }
}

function runGit(args) {
  return execFileSync('git', args, {
    cwd: REPO_ROOT,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  }).trim()
}

function hasCommit(ref) {
  if (!ref || ZERO_SHA.test(ref)) {
    return false
  }

  try {
    runGit(['rev-parse', '--verify', `${ref}^{commit}`])
    return true
  } catch {
    return false
  }
}

function getTagsPointingAtHead() {
  const output = runGit(['tag', '--points-at', 'HEAD', '--list', 'v*', '--sort=-version:refname'])
  return output ? output.split('\n').filter(Boolean) : []
}

function getLatestVersionTag(exclude = new Set()) {
  const output = runGit(['tag', '--list', 'v*', '--sort=-version:refname'])
  const tags = output ? output.split('\n').filter(Boolean) : []
  return tags.find((tag) => !exclude.has(tag)) ?? null
}

function getDiffBase(baseRef, headRef) {
  if (hasCommit(baseRef)) {
    return baseRef
  }

  if (hasCommit(`${headRef}^`)) {
    return `${headRef}^`
  }

  return null
}

function getChangedLineCount(baseRef, headRef) {
  if (!baseRef || !hasCommit(headRef)) {
    return 0
  }

  const summary = runGit(['diff', '--shortstat', baseRef, headRef])
  return extractChangedLineCount(summary)
}

function replaceExactlyOnce(content, pattern, replacement, filePath) {
  const globalPattern = new RegExp(pattern.source, ensureGlobalFlags(pattern.flags))
  const matches = [...content.matchAll(globalPattern)]

  if (matches.length !== 1) {
    throw new Error(`Expected exactly one version entry in ${filePath}, found ${matches.length}`)
  }

  return content.replace(pattern, replacement)
}

export function writeVersionFiles(version, rootDir = REPO_ROOT) {
  const packageJsonPath = path.join(rootDir, 'package.json')
  const packageLockPath = path.join(rootDir, 'package-lock.json')
  const cargoTomlPath = path.join(rootDir, 'src-tauri', 'Cargo.toml')
  const cargoLockPath = path.join(rootDir, 'src-tauri', 'Cargo.lock')
  const tauriConfigPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json')

  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'))
  packageJson.version = version
  fs.writeFileSync(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`)

  const packageLock = JSON.parse(fs.readFileSync(packageLockPath, 'utf8'))
  packageLock.version = version
  if (packageLock.packages?.['']) {
    packageLock.packages[''].version = version
  }
  fs.writeFileSync(packageLockPath, `${JSON.stringify(packageLock, null, 2)}\n`)

  const tauriConfig = JSON.parse(fs.readFileSync(tauriConfigPath, 'utf8'))
  tauriConfig.version = version
  fs.writeFileSync(tauriConfigPath, `${JSON.stringify(tauriConfig, null, 2)}\n`)

  const cargoToml = fs.readFileSync(cargoTomlPath, 'utf8')
  fs.writeFileSync(
    cargoTomlPath,
    replaceExactlyOnce(cargoToml, /^(version = ")([^"]+)"/m, `$1${version}"`, cargoTomlPath),
  )

  const cargoLock = fs.readFileSync(cargoLockPath, 'utf8')
  fs.writeFileSync(
    cargoLockPath,
    replaceExactlyOnce(
      cargoLock,
      /(\[\[package\]\]\s+name = "schedule-reminder"\s+version = ")([^"]+)"/m,
      `$1${version}"`,
      cargoLockPath,
    ),
  )
}

export function resolveReleaseVersion({
  baseRef,
  headRef = 'HEAD',
  thresholds,
} = {}) {
  const headTags = getTagsPointingAtHead()
  if (headTags.length > 0) {
    return {
      version: headTags[0].slice(1),
      tag: headTags[0],
      previousTag: headTags[0],
      bumpType: 'none',
      changedLines: 0,
      alreadyTagged: true,
      initialRelease: false,
    }
  }

  const latestTag = getLatestVersionTag()
  if (!latestTag) {
    return {
      version: '1.0.0',
      tag: 'v1.0.0',
      previousTag: null,
      bumpType: 'initial',
      changedLines: 0,
      alreadyTagged: false,
      initialRelease: true,
    }
  }

  const parsedVersion = parseVersionTag(latestTag)
  if (!parsedVersion) {
    throw new Error(`Unsupported version tag format: ${latestTag}`)
  }

  const diffBase = getDiffBase(baseRef, headRef)
  const changedLines = getChangedLineCount(diffBase, headRef)
  const bumpType = determineBumpType(changedLines, thresholds)
  const nextVersion = formatVersion(incrementVersion(parsedVersion, bumpType))

  return {
    version: nextVersion,
    tag: `v${nextVersion}`,
    previousTag: latestTag,
    bumpType,
    changedLines,
    alreadyTagged: false,
    initialRelease: false,
  }
}

function parseArgs(argv) {
  const options = {
    baseRef: undefined,
    headRef: 'HEAD',
    format: 'json',
    rootDir: REPO_ROOT,
    write: false,
    thresholds: {
      patchMaxLines: process.env.RELEASE_PATCH_MAX_LINES,
      minorMaxLines: process.env.RELEASE_MINOR_MAX_LINES,
    },
  }

  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index]

    switch (argument) {
      case '--base-ref':
        if (!argv[index + 1]) {
          throw new Error('Missing value for --base-ref')
        }
        options.baseRef = argv[index + 1]
        index += 1
        break
      case '--head-ref':
        if (!argv[index + 1]) {
          throw new Error('Missing value for --head-ref')
        }
        options.headRef = argv[index + 1]
        index += 1
        break
      case '--format':
        if (!argv[index + 1]) {
          throw new Error('Missing value for --format')
        }
        options.format = argv[index + 1]
        index += 1
        break
      case '--root-dir':
        if (!argv[index + 1]) {
          throw new Error('Missing value for --root-dir')
        }
        options.rootDir = path.resolve(argv[index + 1])
        index += 1
        break
      case '--write':
        options.write = true
        break
      default:
        throw new Error(`Unknown argument: ${argument}`)
    }
  }

  return options
}

function emitResult(result, format) {
  if (format === 'github-output') {
    for (const [key, value] of Object.entries(result)) {
      console.log(`${key}=${value ?? ''}`)
    }
    return
  }

  console.log(JSON.stringify(result, null, 2))
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const options = parseArgs(process.argv.slice(2))
  const result = resolveReleaseVersion({
    baseRef: options.baseRef,
    headRef: options.headRef,
    thresholds: options.thresholds,
  })

  if (options.write) {
    writeVersionFiles(result.version, options.rootDir)
  }

  emitResult(result, options.format)
}
