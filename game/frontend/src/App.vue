<script setup lang="ts">

import init, { start_application } from '../wasm/game'

init().then(() => {
  start_application('#game')
})

</script>

<template>
  <div>
    <canvas id="game"/>
  </div>
</template>
