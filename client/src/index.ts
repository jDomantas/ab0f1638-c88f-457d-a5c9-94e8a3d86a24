import { Game, World } from "game";
import { NetworkHandler } from "network";

const handler = new NetworkHandler();
console.log(handler.onWorldState);
const game = new Game();
let world: World;
let currentFrame: number;

handler.onWorldState = worldState => {
    console.debug("Initial world state:", world);
    const buffer = game.allocateBuffer(0);
    buffer.putData(worldState.world);
    world = game.deserializeWorld(buffer);
    buffer.free();
    currentFrame = worldState.frame;
    sendInput(currentFrame + 1);
};

handler.onPlayerInputs = inputs => {
    console.log("Player inputs:", inputs);
    inputs.removed_players.forEach(numericId => {
        const id = game.createPlayerId(numericId);
        world = game.removePlayer(id, world);
        id.free();
    });
    world = game.updateWorld(world);
    const ids = Object.keys(inputs.inputs).map(id => parseInt(id, 10));
    ids.sort((a, b) => a - b);
    ids.forEach(numericId => {
        const id = game.createPlayerId(numericId);
        const inputBuffer = game.allocateBuffer(0);
        inputBuffer.putData(inputs.inputs[numericId.toString()]);
        const input = game.deserializeInput(inputBuffer);
        inputBuffer.free();
        world = game.updatePlayer(world, id, input);
        input.free();
        id.free();
    });
    inputs.new_players.forEach(numericId => {
        const id = game.createPlayerId(numericId);
        world = game.addPlayer(id, world);
        id.free();
    });
    currentFrame = inputs.frame;
    sendInput(inputs.frame + 1);
};

function sendInput(frame: number) {
    console.log(`Sending input for frame ${frame}`);
    handler.sendInput({
        frame,
        input: "foobar",
    });
}

console.log(handler.onWorldState);
