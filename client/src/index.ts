import { Client } from "client";
import { Game, PlayerId } from "game";
import { NetworkHandler } from "network";

const canvas = document.getElementById("game-canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d") as CanvasRenderingContext2D;


function showColor(color: number): string {
    function showHex(n: number): string {
        n = n & 0xFF;
        const chars = "0123456789ABCDEF";
        return chars[n >> 4] + chars[n & 0xF];
    }
    return "#" + showHex(color >> 16) + showHex(color >> 8) + showHex(color);
}

const imports = {
    env: {
        draw_rectangle: (x: number, y: number, width: number, height: number, color: number) => {
            ctx.fillStyle = showColor(color);
            ctx.fillRect(x, y, width, height);
        },
        log_str: (ptr: number, len: number) => {},
    },
};

const keyState = {
    letters: 0,
    other: 0,
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
            client.runGameLoop();
        };

        handler.onPlayerInputs = inputs => {
            if (inputs.frame <= client.currentFrameNumber) {
                return;
            }
            client.step(inputs);
            sendInput(inputs.frame + 1);
        };

        function sendInput(frame: number) {
            console.debug(`Sending input for frame ${frame}`);
            const input = game.createInput(keyState.letters, 0, keyState.other, 0);
            const serialized = game.serializeInput(input);
            input.free();
            handler.sendInput({
                frame,
                input: Array.prototype.slice.call(serialized),
            });
        }
    });

function keyIndex(key: string): number {
    if (key == "a") return 0;
    if (key == "b") return 1;
    if (key == "c") return 2;
    if (key == "d") return 3;
    if (key == "e") return 4;
    if (key == "f") return 5;
    if (key == "g") return 6;
    if (key == "h") return 7;
    if (key == "i") return 8;
    if (key == "j") return 9;
    if (key == "k") return 10;
    if (key == "l") return 11;
    if (key == "m") return 12;
    if (key == "n") return 13;
    if (key == "o") return 14;
    if (key == "p") return 15;
    if (key == "q") return 16;
    if (key == "r") return 17;
    if (key == "s") return 18;
    if (key == "t") return 19;
    if (key == "u") return 20;
    if (key == "v") return 21;
    if (key == "w") return 22;
    if (key == "x") return 23;
    if (key == "y") return 24;
    if (key == "z") return 25;
    if (key == "ArrowUp") return 32;
    if (key == "ArrowDown") return 33;
    if (key == "ArrowLeft") return 34;
    if (key == "ArrowRight") return 35;
    if (key == "0") return 36;
    if (key == "1") return 37;
    if (key == "2") return 38;
    if (key == "3") return 39;
    if (key == "4") return 40;
    if (key == "5") return 41;
    if (key == "6") return 42;
    if (key == "7") return 43;
    if (key == "8") return 44;
    if (key == "9") return 45;
    return -1;
}

document.onkeydown = (ev) => {
    if([32, 37, 38, 39, 40].indexOf(ev.keyCode) > -1) {
        ev.preventDefault();
    }
    const index = keyIndex(ev.key);
    if (index >= 32) {
        keyState.other |= (1 << (index - 32));
    } else if (index >= 0) {
        keyState.other |= (1 << index);
    }
};

document.onkeyup = (ev) => {
    const index = keyIndex(ev.key);
    if (index >= 32) {
        keyState.other &= ~(1 << (index - 32));
    } else if (index >= 0) {
        keyState.other &= ~(1 << index);
    }
};

document.onblur = (ev) => {
    keyState.letters = 0;
    keyState.other = 0;
};
