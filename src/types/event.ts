type Events = {
  "data:weight": number;
  "error:componentFailed": string;
};

export type Event = keyof Events;
export type Payload<T extends Event> = Events[T];
