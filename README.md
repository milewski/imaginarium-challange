<h1 align="center">Imaginarium</h1>
<p align="center">
Build. Explore. Leave Your Mark.
</p>
<p align="center"><img width="1000" src="https://github.com/user-attachments/assets/f298f769-dd71-4e09-b95a-0d8261e1eb6c"></p>


https://github.com/user-attachments/assets/7890823f-c500-458e-9061-b4435e316555


This is the main repository for my submission to [Alibaba Cloud Web Game Challenge](https://dev.to/challenges/alibaba).

The project is organized into three folders:

- **[Game](./game)**: The core game, built with Bevy and compiled to WebAssembly (WASM) to run in the browser.
- **[Frontend](./game/frontend)**: A thin Vue.js wrapper that handles loading the WASM file, managing modals, audio tokens, and browser-to-game communication.
- **[Backend](./server)**: A WebSocket server and API, built in Rust, responsible for real-time communication with the frontend and interfacing with a ComfyUI instance tunneled securely via Tailscale.
- **[Infrastructure](./infrastructure)**: Terraform configurations and stack files for deploying the application on a Docker Swarm cluster, ensuring easy scaling and efficient resource management.

For a more detailed overview, including screenshots, you can read the submission sent to the challenge here:

https://dev.to/milewski/imaginarium-build-explore-leave-your-mark-1p3i
