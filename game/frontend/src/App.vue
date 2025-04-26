<script setup lang="ts">

    import init, { start_application } from '../wasm/game'
    import MonumentPrompt from '@/components/MonumentPrompt.vue'
    import { ref } from 'vue'

    const started = ref(false)

    function startGame() {

        started.value = true

        init()
            .then(() => start_application('#game'))
            .catch(error => {
                if (!error.message.startsWith('Using exceptions for control flow')) {
                    throw error
                }
            })

    }

</script>

<template>

    <div>

        <button @click="startGame" v-if="started === false">
            start
        </button>

        <template v-else>

            <MonumentPrompt/>

            <canvas
                id="game"
                class="rounded-xl outline-blue-300 outline-4 hover:outline-8 transition-all outline-offset-4"/>

        </template>

    </div>

</template>
