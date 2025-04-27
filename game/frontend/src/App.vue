<script setup lang="ts">

    import init, { start_application } from '../wasm/game'
    import MonumentPrompt from '@/components/MonumentPrompt.vue'
    import { ref } from 'vue'
    import LoadingScreen from '@/components/LoadingScreen.vue'
    import { TransitionScale } from '@morev/vue-transitions'
    import AnimatedBackground from '@/components/AnimatedBackground.vue'

    const loading = ref(true)
    const shouldStart = ref(false)

    async function minimum<T>(promise: Promise<T>, delay: number): Promise<T> {
        return await Promise.all([ promise, new Promise(resolve => setTimeout(resolve, delay)) ]).then(([ result ]) => result)
    }

    minimum(init(), 1000).then(() => loading.value = false)

    function startGame() {
        shouldStart.value = true
        try {
            start_application('#game')
        } catch (error: any) {
            if (!error.message.startsWith('Using exceptions for control flow')) {
                throw error
            }
        }
    }

</script>

<template>

    <div v-show="!loading" class="relative min-w-[1216px] min-h-[684px]">

        <MonumentPrompt/>

        <div class="absolute flex justify-center items-center w-full left-0 right-0 top-0 bottom-0 mx-auto">

            <button
                @click="startGame"
                class="align-middle px-12 rounded-[50px] w-[374px] h-[80px] bg-amber-300 z-10 transition-all duration-700"
                :class="{ 'w-[1216px] h-[684px] rounded-xl': shouldStart }">

                    <span class="transition-all duration-700 font-mono text-2xl font-bold "
                          :class="{ 'opacity-0': shouldStart, 'opacity-100': !shouldStart }">
                        Click here to start
                    </span>

            </button>

        </div>

        <TransitionScale :delay="500">

            <canvas
                v-show="shouldStart"
                id="game"
                width="1216"
                height="684"
                class="rounded-xl outline-amber-300 outline-4 hover:outline-8 transition-all outline-offset-4 z-50 relative"/>

        </TransitionScale>

    </div>

    <LoadingScreen v-if="loading"/>

    <AnimatedBackground/>

</template>
