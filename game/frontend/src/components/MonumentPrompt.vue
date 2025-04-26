<script setup lang="ts">

    import { Drawer, DrawerContent, DrawerDescription, DrawerFooter, DrawerHeader, DrawerTitle } from '@/components/ui/drawer'
    import { ref } from 'vue'
    import { registerFunction } from '@/RustBridge.ts'
    import { Textarea } from '@/components/ui/textarea'
    import { Button } from '@/components/ui/button'

    const open = ref(false)
    const prompt = ref('')

    const send = registerFunction<string | null>('show_modal', function () {
        open.value = true
    })

    function build() {
        send(prompt.value ? prompt.value : null)
        open.value = false
        prompt.value = ''
    }

    function closeConnection() {
        send(null)
        open.value = false
        prompt.value = ''
    }

</script>

<template>

    <Drawer :open="open" dismissible @close="closeConnection">

        <DrawerContent class="mx-auto w-1/2 px-10 text-center">

            <DrawerHeader class="flex justify-center items-center gap-4" @keydown.esc="closeConnection">

                <img src="../assets/giraffe.png" :width="100" alt="" class="mb-5 select-none">

                <DrawerTitle class="text-4xl">Build a Monument!</DrawerTitle>

                <DrawerDescription>
                    Describe the monument you want to create — the more vivid and detailed, the better!<br/>
                    We'll bring your vision to life using AI.
                </DrawerDescription>

                <Textarea v-model="prompt" :maxlength="500" :rows="10" class="max-h-44"/>

            </DrawerHeader>

            <DrawerFooter>

                <Button size="lg" @click="build" :disabled="!prompt">
                    Create Monument
                </Button>

                <Button size="sm" variant="outline" @click="closeConnection">
                    Cancel Build
                </Button>

            </DrawerFooter>

        </DrawerContent>

    </Drawer>

</template>