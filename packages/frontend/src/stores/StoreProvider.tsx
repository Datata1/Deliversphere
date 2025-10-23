import React, { createContext, useContext, ReactNode } from 'react';
import { RootStore, rootStore } from './RootStore';

const StoreContext = createContext<RootStore | undefined>(undefined);

export const StoreProvider = ({ children }: { children: ReactNode }) => {
  return (
    <StoreContext.Provider value={rootStore}>
      {children}
    </StoreContext.Provider>
  );
};

export const useStore = () => {
  return useContext(StoreContext);
};