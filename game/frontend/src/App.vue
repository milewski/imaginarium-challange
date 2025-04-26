<script setup lang="ts">

    import init, { start_application } from '../wasm/game'
    import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog'
    import { Textarea } from '@/components/ui/textarea'
    import { Button } from '@/components/ui/button'
    import { ref } from 'vue'

    init()
        .then(() => start_application('#game'))
        .catch(error => {
            if (!error.message.startsWith('Using exceptions for control flow')) {
                throw error
            }
        })

    const open = ref(false)

    let resolver: ((result: string) => void) | null = null

    window.show_modal = () => {
        open.value = true
        console.log('should open')
        return new Promise((resolve) => {
            resolver = resolve
        })
    }

    function sendBackToRust() {
        if (resolver) {
            resolver('we are ready')
            open.value = false
        }
    }

</script>

<template>

    <div>

        <Dialog :open="open">
            <DialogContent class="sm:max-w-[425px]">
                <DialogHeader>
                    <DialogTitle>Edit profile</DialogTitle>
                    <DialogDescription>
                        Make changes to your profile here. Click save when you're done.
                    </DialogDescription>
                </DialogHeader>
                <div class="grid gap-4 py-4">
                    <Textarea/>
                </div>
                <DialogFooter>
                    <Button @click="sendBackToRust">
                        Save changes
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>

        <canvas id="game" class="rounded-xl outline-blue-300 outline-4 hover:outline-8 transition-all outline-offset-4"/>

    </div>

</template>
