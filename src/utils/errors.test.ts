import { extractErrorMessage } from './errors'

describe('extractErrorMessage', () => {
  it('returns message from Error instances', () => {
    expect(extractErrorMessage(new Error('数据库连接失败'), '默认错误')).toBe('数据库连接失败')
  })

  it('returns message from tauri-style plain objects', () => {
    expect(extractErrorMessage({ message: '提醒模板加载失败' }, '默认错误')).toBe('提醒模板加载失败')
  })

  it('uses fallback for non-error string rejections', () => {
    expect(extractErrorMessage('bad', '默认错误')).toBe('默认错误')
  })
})
