import { createFromRoot } from 'codama'
import { renderVisitor } from '@codama/renderers-js'
import { rootNodeFromAnchor } from '@codama/nodes-from-anchor'
import { readFileSync } from 'fs'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))

// Read the Anchor IDL
const idlPath = join(__dirname, '../../private-transfer/target/idl/private_transfer.json')
const idl = JSON.parse(readFileSync(idlPath, 'utf-8'))

// Create Codama tree from Anchor IDL
const codama = createFromRoot(rootNodeFromAnchor(idl))

// Generate Kit-compatible TypeScript client
const outputDir = join(__dirname, '../src/generated')
console.log(`Generating client to ${outputDir}...`)

codama.accept(
  renderVisitor(outputDir, {
    formatCode: true,
  })
)

console.log('Client generated successfully!')