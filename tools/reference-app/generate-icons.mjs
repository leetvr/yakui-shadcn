// Turn pinned Lucide vector assets into ordinary Rust constants. No SVG parser,
// browser or Javascript runtime is needed by the native component library.
import {readFile, writeFile, mkdir, cp, readdir} from 'node:fs/promises'
import {resolve, dirname} from 'node:path'
import {fileURLToPath} from 'node:url'
import {createHash} from 'node:crypto'
import {execFileSync} from 'node:child_process'
import * as icons from 'lucide-react'
import svgpath from 'svgpath'

const root=resolve(dirname(fileURLToPath(import.meta.url)),'../..')
const upstream=resolve(process.argv[2]||'/private/tmp/yakui-shadcn-upstream')
const pin=JSON.parse(await readFile(resolve(root,'reference/upstream.json'),'utf8')).shadcn.commit
if(execFileSync('git',['-C',upstream,'rev-parse','HEAD'],{encoding:'utf8'}).trim()!==pin)throw new Error('Unexpected shadcn revision')
const version=JSON.parse(await readFile(new URL('./node_modules/lucide-react/package.json',import.meta.url),'utf8')).version
if(version!=='0.474.0')throw new Error(`Unexpected Lucide version ${version}`)
const names=new Set(['Check','ChevronDown','ChevronRight','ChevronLeft','Plus','X','Search','ArrowRight','Loader','LoaderCircle','Menu','Sun','Moon','Mail','Settings','MoreHorizontal'])
const sources={}
for(const folder of ['apps/v4/examples/base','apps/v4/registry/bases/base/ui']){
 for(const file of (await readdir(resolve(upstream,folder))).filter(file=>file.endsWith('.tsx'))){
  const path=`${folder}/${file}`, source=await readFile(resolve(upstream,path),'utf8')
  const imports=[...source.matchAll(/import\s*\{([^}]+)\}\s*from\s*["']lucide-react["']/gs)]
  const found=imports.flatMap(match=>match[1].split(',').map(name=>name.trim().replace(/^type\s+/, '').split(/\s+as\s+/)[0]).filter(Boolean))
  found.push(...[...source.matchAll(/lucide="([^"]+)"/g)].map(match=>match[1]))
  for(const imported of found){
   if(['LucideIcon','Icon','IconNode'].includes(imported))continue
   const name=imported.replace(/Icon$/,'')
   if(!icons[imported]&&!icons[name])throw new Error(`Missing pinned Lucide ${imported} in ${path}`)
   names.add(name);(sources[name]??=[]).push(path)
  }
 }
}

const distance=(a,b)=>Math.hypot(a[0]-b[0],a[1]-b[1])
function flatten(d){
 const result=[];let points=[],start=[0,0]
 const flush=()=>{if(points.length>1)result.push(points);points=[]}
 const path=svgpath(d).abs().unshort().unarc()
 if(path.err)throw new Error(path.err)
 path.iterate((segment,index,x,y)=>{
  const command=segment[0], p=[x,y]
  if(command==='M'){flush();start=segment.slice(1);points=[start]}
  else if(command==='L')points.push(segment.slice(1))
  else if(command==='H')points.push([segment[1],y])
  else if(command==='V')points.push([x,segment[1]])
  else if(command==='Z'||command==='z'){if(distance(points.at(-1),start)>1e-6)points.push(start);flush()}
  else if(command==='C'||command==='Q'){
   const controls=command==='C'?[p,segment.slice(1,3),segment.slice(3,5),segment.slice(5,7)]:[p,segment.slice(1,3),segment.slice(3,5)]
   const length=controls.slice(1).reduce((sum,point,i)=>sum+distance(controls[i],point),0)
   const count=Math.max(1,Math.ceil(length/0.35))
   for(let step=1;step<=count;step++){
    const t=step/count,u=1-t
    const weights=command==='C'?[u*u*u,3*u*u*t,3*u*t*t,t*t*t]:[u*u,2*u*t,t*t]
    points.push([0,1].map(axis=>controls.reduce((sum,point,i)=>sum+point[axis]*weights[i],0)))
   }
  } else throw new Error(`Unhandled normalized SVG command ${command}`)
 })
 flush();return result
}
function pathsFor(type,a){
 const n=key=>Number(a[key]||0)
 if(type==='path')return flatten(a.d)
 if(type==='line')return [[[n('x1'),n('y1')],[n('x2'),n('y2')]]]
 if(type==='polyline'||type==='polygon'){
  const values=a.points.match(/[-+]?(?:\d*\.)?\d+(?:e[-+]?\d+)?/gi).map(Number)
  const points=[];for(let i=0;i<values.length;i+=2)points.push(values.slice(i,i+2))
  if(type==='polygon')points.push(points[0]);return [points]
 }
 if(a.fill && !(type==='circle' && a.fill==='currentColor' && n('r')<=1))throw new Error('Unsupported filled Lucide shape')
 if(type==='circle'||type==='ellipse'){
  const x=n('cx'),y=n('cy'),rx=type==='circle'?n('r'):n('rx'),ry=type==='circle'?n('r'):n('ry')
  return flatten(`M ${x+rx} ${y} A ${rx} ${ry} 0 1 1 ${x-rx} ${y} A ${rx} ${ry} 0 1 1 ${x+rx} ${y} Z`)
 }
 if(type==='rect'){
  const x=n('x'),y=n('y'),w=n('width'),h=n('height'),r=Math.min(n('rx'),w/2,h/2)
  if(!r)return flatten(`M ${x} ${y} H ${x+w} V ${y+h} H ${x} Z`)
  return flatten(`M ${x+r} ${y} H ${x+w-r} A ${r} ${r} 0 0 1 ${x+w} ${y+r} V ${y+h-r} A ${r} ${r} 0 0 1 ${x+w-r} ${y+h} H ${x+r} A ${r} ${r} 0 0 1 ${x} ${y+h-r} V ${y+r} A ${r} ${r} 0 0 1 ${x+r} ${y} Z`)
 }
 throw new Error(`Unhandled Lucide SVG element ${type}`)
}
const assets=[]
for(const name of [...names].sort()){
 const component=icons[name]||icons[`${name}Icon`]
 const nodes=component.render({},null).props.iconNode
 const paths=nodes.flatMap(([type,attributes])=>pathsFor(type,attributes))
 if(!paths.length||paths.flat().some(point=>point.length!==2||point.some(value=>!Number.isFinite(value))))throw new Error(`Invalid native paths for ${name}`)
 if(paths.reduce((sum,path)=>sum+path.length*17,0)>65535)throw new Error(`Lucide ${name} exceeds one native mesh`)
 assets.push({name,source_sha256:createHash('sha256').update(JSON.stringify(nodes)).digest('hex'),sources:sources[name]||[],nodes,paths})
}
const float=value=>{const s=String(Number(value.toFixed(5)));return /[.e]/i.test(s)?s:`${s}.0`}
let rust='// Generated from lucide-react 0.474.0 by tools/reference-app/generate-icons.mjs.\n// ISC/Feather MIT notices: ../icons/LICENSE.\n'
rust+='/// Lucide icons used by the pinned first-party component catalogue.\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum Icon {\n'
for(const asset of assets)rust+=`    /// Lucide ${asset.name}.\n    ${asset.name},\n`
rust+='}\nimpl Icon {\n    /// Every icon available in this native catalogue.\n    pub const ALL: &\'static [Self] = &[\n'
for(const asset of assets)rust+=`        Self::${asset.name},\n`
rust+='    ];\n    /// The upstream Lucide name.\n    pub fn name(self) -> &\'static str { match self {\n'
for(const asset of assets)rust+=`        Self::${asset.name} => "${asset.name}",\n`
rust+='    } }\n    pub(crate) fn paths(self) -> &\'static [&\'static [(f32, f32)]] { match self {\n'
for(const asset of assets){rust+=`        Self::${asset.name} => &[\n`;for(const path of asset.paths)rust+=`            &[${path.map(point=>`(${float(point[0])}, ${float(point[1])})`).join(',')}],\n`;rust+='        ],\n'}
rust+='    } }\n}\n'
const crate=resolve(root,'crates/shadcn')
await mkdir(resolve(crate,'icons'),{recursive:true})
await cp(new URL('./node_modules/lucide-react/LICENSE',import.meta.url),resolve(crate,'icons/LICENSE'))
await writeFile(resolve(crate,'icons/provenance.json'),JSON.stringify({shadcn_commit:pin,lucide_version:version,maximum_control_polygon_step:0.35,icons:assets.map(({paths,...asset})=>asset)},null,2)+'\n')
await writeFile(resolve(crate,'src/lucide_icons.rs'),rust)
await mkdir(new URL('./public/',import.meta.url),{recursive:true})
await writeFile(new URL('./public/lucide-catalogue.json',import.meta.url),JSON.stringify(assets.map(asset=>asset.name))+'\n')
execFileSync('rustfmt',[resolve(crate,'src/lucide_icons.rs'),'--edition','2024'])
console.log(`${assets.length} pinned native Lucide icons generated`)
