import React, { useContext } from "react";
import { EmulatorContext } from "./App";

export function Home(): React.JSX.Element {
  const emulator = useContext(EmulatorContext);
  const message = emulator == null ? "Emulator is not ready" : emulator.greet("wasmboy-rs");
  return <h1>{message}</h1>
}
