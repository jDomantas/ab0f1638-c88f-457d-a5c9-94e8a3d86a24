import { Client } from "client";
import { Game, PlayerId } from "game";
import { NetworkHandler } from "network";

WebAssembly
    .instantiateStreaming(fetch("/game/code.wasm"))
    .then(wasm => {
        const handler = new NetworkHandler();
        const game = new Game(wasm.instance);
        let client: Client;

        handler.onWorldState = worldState => {
            console.debug("Initial world state:", worldState);
            const rawWorld = new Uint8Array(JSON.parse(worldState.world));
            const playerId = new PlayerId(worldState.localPlayer);
            client = new Client(game, playerId, worldState.frame, rawWorld);
            sendInput(client.currentFrameNumber + 1);
        };

        handler.onPlayerInputs = inputs => {
            client.step(inputs);
            sendInput(inputs.frame + 1);
        };

        function sendInput(frame: number) {
            console.debug(`Sending input for frame ${frame}`);
            handler.sendInput({
                frame,
                input: "foobar",
            });
        }
    });
