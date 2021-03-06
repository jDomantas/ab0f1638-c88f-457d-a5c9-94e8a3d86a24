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

        this.currentFrame += 1;
    }

    public runGameLoop() {
        const renderLoop = () => {
            window.requestAnimationFrame(renderLoop);
            this.render();
        };
        window.requestAnimationFrame(renderLoop);
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

    private updatePlayer(player: number, serializedInput: Uint8Array) {
        const id = new PlayerId(player);
        const input = this.game.deserializeInput(serializedInput);
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

    private render() {
        this.game.render(this.world, this.localPlayer, 640, 480);
    }
}
