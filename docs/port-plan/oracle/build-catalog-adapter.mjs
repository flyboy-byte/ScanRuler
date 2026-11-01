// SPDX-License-Identifier: AGPL-3.0-only
// Re-export actual original APIs, without changing tracked original sources.
import { writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { execFileSync } from 'node:child_process'
import { reference } from './catalog-reference.mjs'
const { root } = reference(process.argv[2])
writeFileSync(resolve(root, '.port-catalog-entry.ts'), `export { startEngine } from './src/headless/engine'\nexport { listCommands, runsHeadless } from './src/commands/registry'\n`)
writeFileSync(resolve(root, '.port-catalog-build.mjs'), `import { build } from 'vite'\nawait build({root:process.cwd(),configFile:process.cwd()+'/vite.config.ts',logLevel:'warn',publicDir:false,build:{ssr:'.port-catalog-entry.ts',outDir:'dist-catalog',emptyOutDir:true,target:'node22',sourcemap:true,rollupOptions:{output:{entryFileNames:'catalog.mjs',format:'es'}}},ssr:{target:'node'}})\n`)
execFileSync(process.execPath, ['.port-catalog-build.mjs'], { cwd: root, stdio: 'inherit', timeout: 120000 })
reference(root) // fail if any tracked reference file changed during build
