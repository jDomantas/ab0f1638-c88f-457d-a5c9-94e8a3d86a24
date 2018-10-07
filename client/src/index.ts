import { Client } from "client";
import { Game, World } from "game";
import { NetworkHandler } from "network";

const handler = new NetworkHandler();
const game = new Game();
let client: Client;

handler.onWorldState = worldState => {
    console.debug("Initial world state:", worldState);
    const buffer = game.allocateBuffer(0); // FIXME: should be size of received world
    buffer.putData(worldState.world);
    const playerId = game.createPlayerId(worldState.localPlayer);
    client = new Client(game, playerId, worldState.frame, buffer);
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
