import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { execFileSync } from 'node:child_process'

import { describe, expect, it } from 'vitest'

import {
  determineBumpType,
  extractChangedLineCount,
  formatVersion,
  incrementVersion,
  parseVersionTag,
  writeVersionFiles,
} from './release-version.mjs'

const SCRIPT_PATH = path.resolve(process.cwd(), 'scripts', 'release-version.mjs')

describe('release version helpers', () => {
  it('parses semantic version tags', () => {
    expect(parseVersionTag('v2.4.6')).toEqual({ major: 2, minor: 4, patch: 6 })
    expect(parseVersionTag('release-2.4.6')).toBeNull()
  })

  it('classifies change size into semantic bumps', () => {
    expect(determineBumpType(0, { patchMaxLines: 50, minorMaxLines: 200 })).toBe('patch')
    expect(determineBumpType(80, { patchMaxLines: 50, minorMaxLines: 200 })).toBe('minor')
    expect(determineBumpType(260, { patchMaxLines: 50, minorMaxLines: 200 })).toBe('major')
  })

  it('counts only inserted and deleted lines from git shortstat output', () => {
    expect(extractChangedLineCount('1 file changed, 2 insertions(+), 3 deletions(-)')).toBe(5)
    expect(extractChangedLineCount('3 files changed, 12 insertions(+)')).toBe(12)
    expect(extractChangedLineCount('2 files changed, 7 deletions(-)')).toBe(7)
  })

  it('increments versions with reset semantics', () => {
    expect(formatVersion(incrementVersion({ major: 1, minor: 2, patch: 3 }, 'patch'))).toBe('1.2.4')
    expect(formatVersion(incrementVersion({ major: 1, minor: 2, patch: 3 }, 'minor'))).toBe('1.3.0')
    expect(formatVersion(incrementVersion({ major: 1, minor: 2, patch: 3 }, 'major'))).toBe('2.0.0')
  })

  it('updates all tracked version files', () => {
    const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'schedule-reminder-version-'))
    const tauriDir = path.join(tempRoot, 'src-tauri')
    const settingsDir = path.join(tempRoot, 'src', 'pages', 'settings')
    fs.mkdirSync(tauriDir, { recursive: true })
    fs.mkdirSync(settingsDir, { recursive: true })

    fs.writeFileSync(
      path.join(tempRoot, 'package.json'),
      JSON.stringify({ name: 'schedule-reminder', version: '0.1.0' }, null, 2),
    )
    fs.writeFileSync(
      path.join(tempRoot, 'package-lock.json'),
      JSON.stringify(
        {
          name: 'schedule-reminder',
          version: '0.1.0',
          packages: {
            '': {
              name: 'schedule-reminder',
              version: '0.1.0',
            },
          },
        },
        null,
        2,
      ),
    )
    fs.writeFileSync(path.join(tauriDir, 'tauri.conf.json'), JSON.stringify({ version: '0.1.0' }, null, 2))
    fs.writeFileSync(path.join(tauriDir, 'Cargo.toml'), '[package]\nname = "schedule-reminder"\nversion = "0.1.0"\n')
    fs.writeFileSync(
      path.join(tauriDir, 'Cargo.lock'),
      '[[package]]\nname = "schedule-reminder"\nversion = "0.1.0"\n',
    )
    fs.writeFileSync(path.join(settingsDir, 'SettingsPage.tsx'), '时间助手 v0.1.0 · 基于 Tauri 构建\n')

    writeVersionFiles('3.2.1', tempRoot)

    expect(JSON.parse(fs.readFileSync(path.join(tempRoot, 'package.json'), 'utf8')).version).toBe('3.2.1')
    const packageLock = JSON.parse(fs.readFileSync(path.join(tempRoot, 'package-lock.json'), 'utf8'))
    expect(packageLock.version).toBe('3.2.1')
    expect(packageLock.packages[''].version).toBe('3.2.1')
    expect(JSON.parse(fs.readFileSync(path.join(tauriDir, 'tauri.conf.json'), 'utf8')).version).toBe('3.2.1')
    expect(fs.readFileSync(path.join(tauriDir, 'Cargo.toml'), 'utf8')).toContain('version = "3.2.1"')
    expect(fs.readFileSync(path.join(tauriDir, 'Cargo.lock'), 'utf8')).toContain('version = "3.2.1"')
    expect(fs.readFileSync(path.join(settingsDir, 'SettingsPage.tsx'), 'utf8')).toContain('时间助手 v3.2.1')
  })

  it('supports the cli write path for version file updates', () => {
    const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'schedule-reminder-cli-version-'))
    const tauriDir = path.join(tempRoot, 'src-tauri')
    const settingsDir = path.join(tempRoot, 'src', 'pages', 'settings')
    fs.mkdirSync(tauriDir, { recursive: true })
    fs.mkdirSync(settingsDir, { recursive: true })

    fs.writeFileSync(
      path.join(tempRoot, 'package.json'),
      JSON.stringify({ name: 'schedule-reminder', version: '0.1.0' }, null, 2),
    )
    fs.writeFileSync(
      path.join(tempRoot, 'package-lock.json'),
      JSON.stringify(
        {
          name: 'schedule-reminder',
          version: '0.1.0',
          packages: {
            '': {
              name: 'schedule-reminder',
              version: '0.1.0',
            },
          },
        },
        null,
        2,
      ),
    )
    fs.writeFileSync(path.join(tauriDir, 'tauri.conf.json'), JSON.stringify({ version: '0.1.0' }, null, 2))
    fs.writeFileSync(path.join(tauriDir, 'Cargo.toml'), '[package]\nname = "schedule-reminder"\nversion = "0.1.0"\n')
    fs.writeFileSync(
      path.join(tauriDir, 'Cargo.lock'),
      '[[package]]\nname = "schedule-reminder"\nversion = "0.1.0"\n',
    )
    fs.writeFileSync(path.join(settingsDir, 'SettingsPage.tsx'), '时间助手 v0.1.0 · 基于 Tauri 构建\n')

    const output = execFileSync(
      'node',
      [SCRIPT_PATH, '--write', '--root-dir', tempRoot],
      { encoding: 'utf8' },
    )
    const result = JSON.parse(output)

    expect(JSON.parse(fs.readFileSync(path.join(tempRoot, 'package.json'), 'utf8')).version).toBe(result.version)
    expect(fs.readFileSync(path.join(tauriDir, 'Cargo.toml'), 'utf8')).toContain(`version = "${result.version}"`)
    expect(fs.readFileSync(path.join(settingsDir, 'SettingsPage.tsx'), 'utf8')).toContain(`时间助手 v${result.version}`)
  })
})
