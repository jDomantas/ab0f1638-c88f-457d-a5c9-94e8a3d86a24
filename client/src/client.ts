import { Game, PlayerId, World } from "game";
import { PlayerInputMessage } from "network";

export class Client {
    private game: Game;
    private world: World;
    private currentFrame: number;
    private localPlayer: PlayerId;

    public constructor(game: Game, localPlayer: PlayerId, currentFrame: number, worldBuf: Uint8Array) {
        this.game = game;
        this.localPlayer = localPlayer;
        this.currentFrame = currentFrame;
        this.world = this.game.deserializeWorld(worldBuf);
    }

    public get currentFrameNumber(): number {
        return this.currentFrame;
    }

    public step(inputs: PlayerInputMessage) {
        inputs.removedPlayers.forEach(id => this.removePlayer(id));

        this.updateWorld();

        const ids = Object.keys(inputs.inputs).map(id => parseInt(id, 10));
        ids.sort((a, b) => a - b);
        ids.forEach(id => this.updatePlayer(id, inputs.inputs[id.toString()]));

        inputs.newPlayers.forEach(id => this.addPlayer(id));

        this.currentFrame = inputs.frame;
    }

    private addPlayer(player: number) {
        const id = new PlayerId(player);
        const oldWorld = this.world;
        this.world = this.game.addPlayer(id, oldWorld);
        oldWorld.free();
    }

    private removePlayer(player: number) {
        const id = new PlayerId(player);
        const oldWorld = this.world;
        this.world = this.game.removePlayer(id, oldWorld);
        oldWorld.free();
    }

    private updatePlayer(player: number, serializedInput: string) {
        const id = new PlayerId(player);
        const raw = new Uint8Array(JSON.parse(serializedInput));
        const input = this.game.deserializeInput(raw);
        const oldWorld = this.world;
        this.world = this.game.updatePlayer(oldWorld, id, input);
        oldWorld.free();
        input.free();
    }

    private updateWorld() {
        const oldWorld = this.world;
        this.world = this.game.updateWorld(oldWorld);
        oldWorld.free();
    }
}
