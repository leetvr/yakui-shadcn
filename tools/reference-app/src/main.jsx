import * as React from 'react'
import { createRoot } from 'react-dom/client'
import './theme.css'
const p=new URLSearchParams(location.search)
document.documentElement.classList.toggle('dark',p.get('theme')==='dark')
const name=p.get('example')||'button-default'
// Restrict imports to examples whose runtime dependencies are installed.
const loaders=import.meta.glob(['./examples/button-default.tsx','./examples/button-demo.tsx','./examples/button-outline.tsx','./examples/button-secondary.tsx','./examples/button-ghost.tsx','./examples/button-destructive.tsx','./examples/button-link.tsx','./examples/button-size.tsx','./examples/button-icon.tsx','./examples/button-rounded.tsx'])
const load=loaders[`./examples/${name}.tsx`]
if(!load) throw new Error(`Unconfigured reference example ${name}`)
load().then(({default:Example})=>{createRoot(document.getElementById('root')).render(<Example/>);window.referenceReady=true})
