import { Client } from "client";
import { Game, PlayerId } from "game";
import { NetworkHandler } from "network";

const imports = {
    env: {
        draw_rectangle: (x: number, y: number, width: number, height: number, color: number) => {},
        log_str: (ptr: number, len: number) => {},
    },
};

WebAssembly
    .instantiateStreaming(fetch("/game/code.wasm"), imports)
    .then(wasm => {
        const handler = new NetworkHandler();
        const game = new Game(wasm.instance);
        let client: Client;

        handler.onWorldState = worldState => {
            console.debug("Initial world state:", worldState);
            // FIXME: hack, should replace with proper binary messages
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
            const input = game.createInput(0, 0, 0, 0);
            const serialized = game.serializeInput(input);
            input.free();
            handler.sendInput({
                frame,
                input: Array.prototype.slice.call(serialized),
            });
        }
    });
