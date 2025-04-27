import path from 'node:path'
import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

export default defineConfig({
    plugins: [ vue(), tailwindcss() ],
    // build: {
    //     rollupOptions: {
    //         output: {
    //             assetFileNames() {
    //                 return 'assets/[name][extname]'
    //             },
    //         },
    //     },
    // },
    resolve: {
        alias: {
            '@': path.resolve(__dirname, './src'),
        },
    },
})