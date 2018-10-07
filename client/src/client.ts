import { Game, PlayerId, VmBuffer, World } from "game";
import { PlayerInputMessage } from "network";

export class Client {
    private game: Game;
    private world: World;
    private currentFrame: number;
    private localPlayer: PlayerId;

    public constructor(game: Game, localPlayer: PlayerId, currentFrame: number, worldBuf: VmBuffer) {
        this.game = game;
        this.localPlayer = localPlayer;
        this.currentFrame = currentFrame;
        this.world = this.game.deserializeWorld(worldBuf);
        worldBuf.free();
    }

    public step(inputs: PlayerInputMessage) {
        inputs.removed_players.forEach(id => this.removePlayer(id));

        this.updateWorld();

        const ids = Object.keys(inputs.inputs).map(id => parseInt(id, 10));
        ids.sort((a, b) => a - b);
        ids.forEach(id => this.updatePlayer(id, inputs.inputs[id.toString()]));

        inputs.new_players.forEach(id => this.addPlayer(id));

        this.currentFrame = inputs.frame;
    }

    private addPlayer(player: number) {
        const id = this.game.createPlayerId(player);
        const oldWorld = this.world;
        this.world = this.game.addPlayer(id, oldWorld);
        oldWorld.free();
        id.free();
    }

    private removePlayer(player: number) {
        const id = this.game.createPlayerId(player);
        const oldWorld = this.world;
        this.world = this.game.removePlayer(id, oldWorld);
        oldWorld.free();
        id.free();
    }

    private updatePlayer(player: number, serializedInput: string) {
        const id = this.game.createPlayerId(player);
        // TODO: buffer size should depend on serialized input size
        const inputBuffer = this.game.allocateBuffer(0);
        inputBuffer.putData(serializedInput);
        const input = this.game.deserializeInput(inputBuffer);
        inputBuffer.free();
        const oldWorld = this.world;
        this.world = this.game.updatePlayer(oldWorld, id, input);
        oldWorld.free();
        input.free();
        id.free();
    }

    private updateWorld() {
        const oldWorld = this.world;
        this.world = this.game.updateWorld(oldWorld);
        oldWorld.free();
    }
}
