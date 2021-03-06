export class Handle<Kind extends string> {
    public kind: Kind;
    public readonly value: number;

    constructor(value: number, kind: Kind) {
        this.kind = kind;
        this.value = value;
    }
}

export type WorldHandle = Handle<"world">;
export type InputHandle = Handle<"input">;
export type BufferHandle = Handle<"buffer">;

export class LowLevelGame {
    private readonly instance: WebAssembly.Instance;

    public constructor(instance: WebAssembly.Instance) {
        this.instance = instance;
        this.instance.exports.initialize();
    }

    public initialWorld(): WorldHandle {
        const value = this.instance.exports.initial_world();
        return new Handle(value, "world");
    }

    public updateWorld(world: WorldHandle): WorldHandle {
        const value = this.instance.exports.update_world(world.value);
        return new Handle(value, "world");
    }

    public updatePlayer(world: WorldHandle, playerId: number, input: InputHandle): WorldHandle {
        const value = this.instance.exports.update_player(world.value, playerId, input.value);
        return new Handle(value, "world");
    }

    public addPlayer(world: WorldHandle, playerId: number): WorldHandle {
        const value = this.instance.exports.add_player(world.value, playerId);
        return new Handle(value, "world");
    }

    public removePlayer(world: WorldHandle, playerId: number): WorldHandle {
        const value = this.instance.exports.remove_player(world.value, playerId);
        return new Handle(value, "world");
    }

    public allocateBuffer(size: number): BufferHandle {
        const value = this.instance.exports.allocate_buffer(size);
        return new Handle(value, "buffer");
    }

    public freeHandle<Kind extends string>(handle: Handle<Kind>) {
        this.instance.exports.free_handle(handle.value);
    }

    public bufferPtr(buffer: BufferHandle): number {
        return this.instance.exports.buffer_ptr(buffer.value);
    }

    public bufferSize(buffer: BufferHandle): number {
        return this.instance.exports.buffer_size(buffer.value);
    }

    public deserializeWorld(buffer: BufferHandle): WorldHandle {
        const value = this.instance.exports.deserialize_world(buffer.value);
        return new Handle(value, "world");
    }

    public deserializeInput(buffer: BufferHandle): InputHandle {
        const value = this.instance.exports.deserialize_input(buffer.value);
        return new Handle(value, "input");
    }

    public serializeInput(input: InputHandle): BufferHandle {
        const value = this.instance.exports.serialize_input(input.value);
        return new Handle(value, "buffer");
    }

    public readMemory(ptr: number, size: number): Uint8Array {
        return this.memory().slice(ptr, ptr + size);
    }

    public writeMemory(ptr: number, data: Uint8Array) {
        this.memory().set(data, ptr);
    }

    public createInput(letters: number, oldLetters: number, other: number, oldOther: number): InputHandle {
        const value = this.instance.exports.create_input(letters, oldLetters, other, oldOther);
        return new Handle(value, "input");
    }

    public render(world: WorldHandle, localPlayer: number, width: number, height: number) {
        this.instance.exports.render(world.value, localPlayer, width, height);
    }

    private memory(): Uint8Array {
        const memory = this.instance.exports.memory as WebAssembly.Memory;
        return new Uint8Array(memory.buffer);
    }
}
