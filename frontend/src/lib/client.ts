import { createReactQueryHooks } from "@rspc/react-query";
import { ProceduresLegacy as Procedures } from "./bindings";

export const rspc = createReactQueryHooks<Procedures>();
