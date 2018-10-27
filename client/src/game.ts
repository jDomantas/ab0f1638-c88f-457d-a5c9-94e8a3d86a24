import { LowLevelGame, WorldHandle, InputHandle, Handle } from "./lowLevelGame";

export class Game {
    private readonly game: LowLevelGame;

    public constructor(wasmInstance: WebAssembly.Instance) {
        this.game = new LowLevelGame(wasmInstance);
    }

    public updatePlayer(world: World, player: PlayerId, input: Input): World {
        return world;
    }

    public updateWorld(world: World): World {
        return world;
    }

    public addPlayer(id: PlayerId, world: World): World {
        return world;
    }

    public removePlayer(id: PlayerId, world: World): World {
        return world;
    }

    public deserializeInput(raw: Uint8Array): Input {
        let buffer = this.game.allocateBuffer(raw.length);
        let ptr = this.game.bufferPtr(buffer);
        this.game.writeMemory(ptr, raw);
        let inputHandle = this.game.deserializeInput(buffer);
        this.game.freeHandle(buffer);
        return new Input(this.game, inputHandle);
    }

    public deserializeWorld(raw: Uint8Array): World {
        let buffer = this.game.allocateBuffer(raw.length);
        let ptr = this.game.bufferPtr(buffer);
        this.game.writeMemory(ptr, raw);
        let worldHandle = this.game.deserializeWorld(buffer);
        this.game.freeHandle(buffer);
        return new World(this.game, worldHandle);
    }

    public serializeInput(input: Input): Uint8Array {
        let buffer = this.game.serializeInput(input.handle);
        let ptr = this.game.bufferPtr(buffer);
        let size = this.game.bufferSize(buffer);
        let raw = this.game.readMemory(ptr, size);
        this.game.freeHandle(buffer);
        return raw;
    }
}

export class Input {
    public readonly game: LowLevelGame;
    public readonly handle: InputHandle;

    constructor(game: LowLevelGame, handle: InputHandle) {
        this.game = game;
        this.handle = handle;
    }

    public free() {
        this.game.freeHandle(this.handle);
    }
}

export class PlayerId {
    public readonly id: number;

    public constructor(id: number) {
        this.id = id;
    }
}

export class World {
    public readonly game: LowLevelGame;
    public readonly handle: WorldHandle;

    constructor(game: LowLevelGame, handle: WorldHandle) {
        this.game = game;
        this.handle = handle;
    }

    public free() {
        this.game.freeHandle(this.handle);
    }
}
