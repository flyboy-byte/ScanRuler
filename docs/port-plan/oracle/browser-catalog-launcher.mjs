// SPDX-License-Identifier: AGPL-3.0-only
// Copied untracked into the frozen checkout to resolve its own Vite/e2e dependencies.

import { createServer } from 'vite'
import { writeFileSync, mkdirSync } from 'node:fs'
import { resolve } from 'node:path'
const mode = process.argv[2], outputDir = resolve(process.argv[3]), root = process.cwd()
process.env.OUT_DIR = resolve(outputDir, mode + '-diagnostics')
process.env.SHOT_DIR = process.env.OUT_DIR
mkdirSync(process.env.OUT_DIR, { recursive: true })
const server = await createServer({ root, configFile: root + '/vite.config.ts', server: { host: '127.0.0.1', port: 0, strictPort: false, open: false } })
let browser, consoleErrors = [], finish
try {
  await server.listen()
  const url = new URL(server.resolvedUrls.local[0])
  if (mode === 'core') url.searchParams.set('plugins', 'none')
  const helpers = await import('./scripts/e2e-lib.mjs')
  finish = helpers.finish
  const launched = await helpers.launchApp({ url: url.href, agentDataOk: false, protocolTimeout: 120000 })
  browser = launched.browser
  const page = launched.page
  consoleErrors = launched.consoleErrors
  await page.waitForFunction(async (expectedPlugins) => {
    try {
      const reg = await import('/src/commands/registry.ts')
      const plugins = await import('/src/plugins/registry.ts')
      if (!reg.listCommands().some(c => c.name === 'session.state') || plugins.plugins().length !== expectedPlugins) return false
      const state = await reg.runCommand('session.state', {})
      return state?.app?.viewport === true && state.scan === null
    } catch { return false }
  }, { timeout: 120000 }, mode === 'core' ? 0 : 2)
  const data = await page.evaluate(async () => {
    const reg = await import('/src/commands/registry.ts')
    const plugins = await import('/src/plugins/registry.ts')
    const version = await import('/src/version.ts')
    const state = await reg.runCommand('session.state', {})
    const commands = reg.listCommands().map(c => ({ ...c, headless: reg.runsHeadless(c.name) }))
    return { appVersion: version.APP_VERSION, startupViewport: state.app.viewport, scanEmpty: state.scan === null,
      plugins: plugins.plugins().map(p => p.id), commands,
      browserOfferedNames: reg.listCommands().map(c => c.name),
      headlessFlagNames: commands.filter(c => c.headless).map(c => c.name),
      pageOnlyFlagNames: commands.filter(c => !c.headless).map(c => c.name),
      userAgent: navigator.userAgent, language: navigator.language, sourceUrl: location.href }
  })
  data.browserVersion = await browser.version()
  await finish(browser, consoleErrors)
  browser = null
  if (process.exitCode) throw Error('Original browser console/page errors failed source helper')
  data.consoleStatus = 'source-helper-pass'
  writeFileSync(resolve(outputDir, mode + '-raw.json'), JSON.stringify(data, null, 2))
} finally {
  if (browser && finish) await finish(browser, consoleErrors)
  await server.close()
}
