import { Handle, InputHandle, LowLevelGame, WorldHandle } from "./lowLevelGame";

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
        const buffer = this.game.allocateBuffer(raw.length);
        const ptr = this.game.bufferPtr(buffer);
        this.game.writeMemory(ptr, raw);
        const inputHandle = this.game.deserializeInput(buffer);
        this.game.freeHandle(buffer);
        return new Input(this.game, inputHandle);
    }

    public deserializeWorld(raw: Uint8Array): World {
        const buffer = this.game.allocateBuffer(raw.length);
        const ptr = this.game.bufferPtr(buffer);
        this.game.writeMemory(ptr, raw);
        const worldHandle = this.game.deserializeWorld(buffer);
        this.game.freeHandle(buffer);
        return new World(this.game, worldHandle);
    }

    public serializeInput(input: Input): Uint8Array {
        const buffer = this.game.serializeInput(input.handle);
        const ptr = this.game.bufferPtr(buffer);
        const size = this.game.bufferSize(buffer);
        const raw = this.game.readMemory(ptr, size);
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
