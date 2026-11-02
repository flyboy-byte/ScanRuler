// SPDX-License-Identifier: AGPL-3.0-only
import { execFileSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { digest } from './catalog-contract.mjs'
export const candidateRoot = fileURLToPath(new URL('../../../', import.meta.url))
export function reference(rootArg) {
  if (Number(process.versions.node.split('.')[0]) !== 22) throw Error('Reference capture requires Node 22')
  const root = resolve(rootArg)
  const pin = JSON.parse(readFileSync(resolve(candidateRoot, 'docs/port-plan/reference-pin.json'), 'utf8'))
  const git = (...args) => execFileSync('git', ['-C', root, ...args], { encoding: 'utf8' }).trim()
  if (git('rev-parse', 'HEAD') !== pin.commit) throw Error('Reference checkout is not the frozen pin')
  git('diff', '--quiet', 'HEAD', '--') // generated untracked adapters are permitted; source edits are not
  const locks = pin.lockfiles.map(lock => {
    if (git('hash-object', lock.path) !== lock.git_blob_sha) throw Error('Reference lock mismatch: ' + lock.path)
    return { path: lock.path, gitBlob: lock.git_blob_sha, sha256: digest(readFileSync(resolve(root, lock.path))) }
  })
  const pkg = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf8'))
  if (pkg.version !== pin.app_version) throw Error('App version differs from pin')
  return { root, pin, locks, candidate: execFileSync('git', ['rev-parse', 'HEAD'], { cwd: candidateRoot, encoding: 'utf8' }).trim() }
}
