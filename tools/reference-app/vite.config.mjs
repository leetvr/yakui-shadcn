import { defineConfig } from 'vite'
import tailwind from '@tailwindcss/vite'
import { fileURLToPath } from 'node:url'
export default defineConfig({plugins:[tailwind()], esbuild:{jsx:"automatic"}, resolve:{alias:{'@':fileURLToPath(new URL('./src',import.meta.url))}}})
