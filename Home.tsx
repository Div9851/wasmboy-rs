import React, { useContext } from "react";
import { EmulatorContext } from "./App";

export function Home(): React.JSX.Element {
  const emulator = useContext(EmulatorContext);
  const message = emulator == null ? "Emulator is not ready" : emulator.greet("wasmboy-rs");
  const nextFrame = () => {
    if (emulator == null) return;
    emulator.next_frame();
    requestAnimationFrame(nextFrame);
  };
  const handleChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    if (emulator == null || e.target.files == null) return;
    const file = e.target.files[0];
    const rom = await file.arrayBuffer().then((buf) => new Uint8Array(buf));
    emulator.init();
    emulator.load_rom(rom);
    requestAnimationFrame(nextFrame);
  };
  return <>
    <h1>{message}</h1>
    <input type="file" onChange={handleChange} />
  </>
}
