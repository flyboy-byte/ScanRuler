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
let browser, agent, consoleErrors = [], finish, fail
try {
  await server.listen()
  const url = new URL(server.resolvedUrls.local[0])
  if (mode === 'core') url.searchParams.set('plugins', 'none')
  process.env.APP_URL = url.href
  const helpers = await import('./scripts/e2e-lib.mjs')
  finish = helpers.finish
  fail = helpers.fail
  // Original core commands register lazily when the real agent bridge connects.
  // Pair the original SDK/server and browser using the original E2E helpers.
  const { startAgent } = await import('./mcp/tests/client.js')
  const started = await helpers.launchAgent(startAgent)
  agent = started.agent
  url.hash = `agent=${started.port}:${started.token}`
  const launched = await helpers.launchApp({ url: url.href, agentDataOk: true, protocolTimeout: 120000 })
  browser = launched.browser
  const page = launched.page
  consoleErrors = launched.consoleErrors
  const paired = await agent.call('scanruler_wait', { for: 'page', timeoutSeconds: 60 })
  if (paired.error || paired.json?.connected !== true) throw Error('Original MCP bridge did not connect')
  // Probe registry and real viewport state in the page main world.
  const deadline = Date.now() + 120000
  let readiness
  while (Date.now() < deadline) {
    readiness = await page.evaluate(async (expectedPlugins) => {
      try {
        const reg = await import('/src/commands/registry.ts')
        const plugins = await import('/src/plugins/registry.ts')
        const names = reg.listCommands().map(c => c.name)
        if (!names.includes('session.state') || plugins.plugins().length !== expectedPlugins) return { ready: false, names, plugins: plugins.plugins().map(p => p.id) }
        const state = await reg.runCommand('session.state', {})
        return { ready: state?.app?.viewport === true && state.scan === null, viewport: state?.app?.viewport, scanEmpty: state.scan === null, names }
      } catch (e) { return { ready: false, error: String(e) } }
    }, mode === 'core' ? 0 : 2)
    if (readiness.ready) break
    await new Promise(resolve => setTimeout(resolve, 200))
  }
  if (!readiness?.ready) {
    writeFileSync(resolve(process.env.OUT_DIR, 'readiness.json'), JSON.stringify(readiness, null, 2))
    throw Error('Actual page registry/session did not become ready in the fixed startup budget')
  }
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
} catch (error) {
  fail?.('Browser catalog capture failed: ' + error.message)
  throw error
} finally {
  if (browser && finish) await finish(browser, consoleErrors)
  if (agent) await agent.close()
  await server.close()
}
