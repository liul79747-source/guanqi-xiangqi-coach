import { mkdir, copyFile, readFile, stat, writeFile } from 'node:fs/promises'
import { createHash } from 'node:crypto'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..')
const output=path.join(root,'output/观棋')
const files=[
  ['server/target/x86_64-pc-windows-msvc/release/guanqi.exe','guanqi.exe'],
  ...['pikafish/pikafish-windows.exe','pikafish/pikafish.nnue','pikafish/COPYING.txt','models/board.onnx','runtime/onnxruntime.dll','runtime/onnxruntime_providers_shared.dll','runtime/LICENSE.txt'].map(p=>['libs/'+p,'libs/'+p]),
  ['LICENSE','licenses/Apache-2.0.txt'],['THIRD_PARTY_NOTICES.md','licenses/THIRD_PARTY_NOTICES.md'],['README.md','使用说明.md'],
]
const manifest=[]
for(const [from,to] of files){const src=path.join(root,from),dest=path.join(output,to);await stat(src);await mkdir(path.dirname(dest),{recursive:true});await copyFile(src,dest);const bytes=await readFile(dest);manifest.push({file:to,bytes:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex')})}
await writeFile(path.join(output,'manifest.json'),JSON.stringify({product:'观棋',version:'1.0.0',builtAt:new Date().toISOString(),files:manifest},null,2))
console.log(`便携版已整理：${output}\n${manifest.length} 个文件已校验。`)
