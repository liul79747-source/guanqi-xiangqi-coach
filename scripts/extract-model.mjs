// Recover the Apache-2.0 reference project's embedded ONNX resource from its local installation.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs'
const exe=readFileSync(process.argv[2] || 'C:/Program Files/xqlink/xqlink.exe')
const signature=Buffer.from([8,7,18,7,...Buffer.from('pytorch')])
const start=exe.indexOf(signature)
if(start<0) throw new Error('未找到参考程序中的 ONNX 模型')
let offset=start, end=start, graph=false, opset=false
function varint(){let n=0,m=1;for(let i=0;i<8;i++){const b=exe[offset++];n+=(b&127)*m;if(!(b&128))return n;m*=128}throw new Error('Invalid protobuf')}
while(offset<exe.length){
  const saved=offset,tag=varint(),field=Math.floor(tag/8),wire=tag%8
  if(field<1||field>14||![0,2].includes(wire)) break
  if(wire===0)varint(); else {const size=varint(); if(size>exe.length-offset)break;offset+=size}
  if(field===7)graph=true
  if(field===8)opset=true
  end=offset
  if(graph&&opset) break
  if(offset<=saved)throw new Error('Invalid model length')
}
if(!graph||!opset)throw new Error('ONNX graph/opset incomplete')
mkdirSync('libs/models',{recursive:true});writeFileSync('libs/models/board.onnx',exe.subarray(start,end))
console.log(`Extracted ONNX model: ${end-start} bytes`)
