import React, { useState, createContext } from "react"
import wasmInit, { Emulator } from "./core/pkg/gbemu_core"
import { Home } from "./Home"

export const EmulatorContext = createContext<Emulator | null>(null)

export function App(): React.JSX.Element {
  const [emulator, setEmulator] = useState<Emulator | null>(null);
  if (emulator == null) {
    wasmInit().then(() => setEmulator(new Emulator()));
  }
  return (
    <EmulatorContext.Provider value={emulator}>
      <Home />
    </EmulatorContext.Provider>
  )
}
