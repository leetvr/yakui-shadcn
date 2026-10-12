// Diagnostic for native geometry, separate from the documented component previews.
import * as React from 'react'
import * as icons from 'lucide-react'
import names from '../public/lucide-catalogue.json'
export default function LucideCatalogue() {
 return <div style={{display:'flex',flexDirection:'column',gap:8}}>{Array.from({length:Math.ceil(names.length/16)},(_,row)=><div key={row} style={{display:'flex',gap:8}}>{names.slice(row*16,row*16+16).map(name=>{const Icon=icons[name];return <div key={name} title={name} style={{width:48,height:48,display:'flex',alignItems:'center',justifyContent:'center'}}><Icon size={24}/></div>})}</div>)}</div>
}
