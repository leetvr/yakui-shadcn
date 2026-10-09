import { defineConfig } from 'vite'
import tailwind from '@tailwindcss/vite'
import { fileURLToPath } from 'node:url'
export default defineConfig({plugins:[tailwind()], esbuild:{jsx:"automatic"}, optimizeDeps:{include:['react','react-dom/client','react/jsx-runtime','react/jsx-dev-runtime','cn','class-variance-authority','@base-ui/react/button','@base-ui/react/menu','@base-ui/react/avatar','@base-ui/react/checkbox','@base-ui/react/switch','@base-ui/react/radio-group','@base-ui/react/radio','@base-ui/react/separator','@base-ui/react/progress','@base-ui/react/input','@base-ui/react/use-render','@base-ui/react/merge-props']}, resolve:{dedupe:['react','react-dom'],alias:{'@':fileURLToPath(new URL('./src',import.meta.url))}}})
