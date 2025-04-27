<template>
    <svg class="absolute left-0 top-0 -z-50" viewBox="0 0 10 10" xmlns="http://www.w3.org/2000/svg"
         preserveAspectRatio="xMidYMid slice">
        <path d="M 10,-5.25 0,.5"/>
        <path d="M 10,-4.75 0,1"/>
        <path d="M 10,-4.25 0,1.5"/>
        <path d="M 10,-3.75 0,2"/>
        <path d="M 10,-3.25 0,2.5"/>
        <path d="M 10,-2.75 0,3"/>
        <path d="M 10,-2.25 0,3.5"/>
        <path d="M 10,-1.75 0,4"/>
        <path d="M 10,-1.25 0,4.5"/>
        <path d="M 10,-.75 0,5"/>
        <path d="M 10,-.25 0,5.5"/>
        <path d="M 10,.25 0,6"/>
        <path d="M 10,.75 0,6.5"/>
        <path d="M 10,1.25 0,7"/>
        <path d="M 10,1.75 0,7.5"/>
        <path d="M 10,2.25 0,8"/>
        <path d="M 10,2.75 0,8.5"/>
        <path d="M 10,3.25 0,9"/>
        <path d="M 10,3.75 0,9.5"/>
        <path d="M 10,4.25 0,10"/>
        <path d="M 10,4.75 0,10.5"/>
        <path d="M 10,5.25 0,11"/>
        <path d="M 10,5.75 0,11.5"/>
        <path d="M 10,6.25 0,12"/>
        <path d="M 10,6.75 0,12.5"/>
        <path d="M 10,7.25 0,13"/>
        <path d="M 10,7.75 0,13.5"/>
        <path d="M 10,8.25 0,14"/>
        <path d="M 10,8.75 0,14.5"/>
        <path d="M 10,9.25 0,15"/>
        <path d="M 10,9.75 0,15.5"/>
        <path d="M 0,9.75 10,15.5"/>
        <path d="M 0,9.25 10,15"/>
        <path d="M 0,8.75 10,14.5"/>
        <path d="M 0,8.25 10,14"/>
        <path d="M 0,7.75 10,13.5"/>
        <path d="M 0,7.25 10,13"/>
        <path d="M 0,6.75 10,12.5"/>
        <path d="M 0,6.25 10,12"/>
        <path d="M 0,5.75 10,11.5"/>
        <path d="M 0,5.25 10,11"/>
        <path d="M 0,4.75 10,10.5"/>
        <path d="M 0,4.25 10,10"/>
        <path d="M 0,3.75 10,9.5"/>
        <path d="M 0,3.25 10,9"/>
        <path d="M 0,2.75 10,8.5"/>
        <path d="M 0,2.25 10,8"/>
        <path d="M 0,1.75 10,7.5"/>
        <path d="M 0,1.25 10,7"/>
        <path d="M 0,.75 10,6.5"/>
        <path d="M 0,.25 10,6"/>
        <path d="M 0,-.25 10,5.5"/>
        <path d="M 0,-.75 10,5"/>
        <path d="M 0,-1.25 10,4.5"/>
        <path d="M 0,-1.75 10,4"/>
        <path d="M 0,-2.25 10,3.5"/>
        <path d="M 0,-2.75 10,3"/>
        <path d="M 0,-3.25 10,2.5"/>
        <path d="M 0,-3.75 10,2"/>
        <path d="M 0,-4.25 10,1.5"/>
        <path d="M 0,-4.75 10,1"/>
        <path d="M 0,-5.25 10,.5"/>
        <path d="M 0,-5.75 10,0"/>
    </svg>
</template>

<style lang="scss">
    body {
        background-color: #171717;
        background: radial-gradient(#2a2a2a, #000000);
    }

    $t: 12s;

    path {
        stroke: rgba(255, 255, 255, 0.05);
        stroke-width: .005px;
        stroke-linecap: square;
        animation: draw $t linear infinite;
        stroke-dasharray: 30 10
    }

    @keyframes draw {
        80% {
            stroke: rgba(255, 255, 255, 0.15);
        }
        100% {
            stroke-dashoffset: 40;
        }
    }

    svg {
        width: 100vw;
        max-height: 100vh;
    }

    path {
        &:nth-child(2n) {
            animation-delay: -$t*2.25;
            animation-duration: $t*1.27;
        }

        &:nth-child(3n) {
            animation-duration: $t*3.75;
            animation-delay: -$t*1.5;
        }

        &:nth-child(4n) {
            animation-delay: -$t*3.75;
        }

        &:nth-child(5n) {
            animation-delay: -$t*7.5;
            animation-duration: $t*.5;
        }

        &:nth-child(6n) {
            animation-delay: -$t*7.5;
            animation-duration: $t*1.2;
        }

        &:nth-child(7n) {
            animation-delay: -$t*.5;
            animation-duration: $t*1.82;
        }

        &:nth-child(8n) {
            animation-duration: $t*5.3;
            animation-delay: -$t*4.33;
        }
    }

</style>
