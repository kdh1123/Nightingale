import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

// Vitest runs without `globals`, so Testing Library cannot register its own cleanup hook and
// rendered trees would otherwise leak into the next test.
afterEach(cleanup);
