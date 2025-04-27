<script setup lang="ts">

    import { Drawer, DrawerContent, DrawerDescription, DrawerFooter, DrawerHeader, DrawerTitle } from '@/components/ui/drawer'
    import { Textarea } from '@/components/ui/textarea'
    import { Button } from '@/components/ui/button'

</script>

<template>

    <div class="tetrominos z-50">
        <div class="tetromino box1"></div>
        <div class="tetromino box2"></div>
        <div class="tetromino box3"></div>
        <div class="tetromino box4"></div>
    </div>

    <div class="z-50 m-40 absolute left-0 right-0 text-2xl font-bold text-amber-400">... Loading ...</div>

</template>

<style lang="scss">
    $w: 96px;
    $h: 112px;
    $x-space: calc($w / 2);
    $y-space: calc($h / 4) - 1;
    $speed: 1.2s;

    .tetrominos {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-$h, -$w);
    }

    .tetromino {
        width: $w;
        height: $h;
        position: absolute;
        transition: all ease .3s;
        background: url('data:image/svg+xml;utf-8,%3Csvg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 612 684"%3E%3Cpath fill="%23ffd230" d="M305.7 0L0 170.9v342.3L305.7 684 612 513.2V170.9L305.7 0z"/%3E%3Cpath fill="%23fff" d="M305.7 80.1l-233.6 131 233.6 131 234.2-131-234.2-131"/%3E%3C/svg%3E') no-repeat top center;
    }

    .box1 {
        animation: tetromino1 $speed ease-out infinite;
    }

    .box2 {
        animation: tetromino2 $speed ease-out infinite;
    }

    .box3 {
        animation: tetromino3 $speed ease-out infinite;
        z-index: 2;
    }

    .box4 {
        animation: tetromino4 $speed ease-out infinite;
    }

    @keyframes tetromino1 {
        0%, 40% {
            transform: translate(0, 0);
        }
        50% {
            transform: translate($x-space, -$y-space);
        }
        60%, 100% {
            transform: translate($x-space*2, 0);
        }
    }

    @keyframes tetromino2 {
        0%, 20% {
            transform: translate($x-space*2, 0px);
        }
        40%, 100% {
            transform: translate($x-space*3, $y-space);
        }
    }

    @keyframes tetromino3 {
        0% {
            transform: translate($x-space*3, $y-space);
        }
        20%, 60% {
            transform: translate($x-space*2, $y-space*2);
        }
        90%, 100% {
            transform: translate($x-space, $y-space);
        }
    }

    @keyframes tetromino4 {
        0%, 60% {
            transform: translate($x-space, $y-space);
        }
        90%, 100% {
            transform: translate(0, 0);
        }
    }
</style>

