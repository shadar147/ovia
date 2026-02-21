import { describe, it, expect } from "vitest";
import { getPluralForm } from "../plurals";

describe("getPluralForm", () => {
  describe("English", () => {
    it("returns 'one' for 1", () => {
      expect(getPluralForm(1, "en")).toBe("one");
    });

    it("returns 'many' for 0", () => {
      expect(getPluralForm(0, "en")).toBe("many");
    });

    it("returns 'many' for 2+", () => {
      expect(getPluralForm(2, "en")).toBe("many");
      expect(getPluralForm(5, "en")).toBe("many");
      expect(getPluralForm(100, "en")).toBe("many");
    });
  });

  describe("Russian", () => {
    it("returns 'one' for 1, 21, 31, 101", () => {
      expect(getPluralForm(1, "ru")).toBe("one");
      expect(getPluralForm(21, "ru")).toBe("one");
      expect(getPluralForm(31, "ru")).toBe("one");
      expect(getPluralForm(101, "ru")).toBe("one");
    });

    it("returns 'few' for 2, 3, 4, 22, 23, 24", () => {
      expect(getPluralForm(2, "ru")).toBe("few");
      expect(getPluralForm(3, "ru")).toBe("few");
      expect(getPluralForm(4, "ru")).toBe("few");
      expect(getPluralForm(22, "ru")).toBe("few");
      expect(getPluralForm(23, "ru")).toBe("few");
      expect(getPluralForm(24, "ru")).toBe("few");
    });

    it("returns 'many' for 0, 5-20, 25-30", () => {
      expect(getPluralForm(0, "ru")).toBe("many");
      expect(getPluralForm(5, "ru")).toBe("many");
      expect(getPluralForm(11, "ru")).toBe("many");
      expect(getPluralForm(12, "ru")).toBe("many");
      expect(getPluralForm(14, "ru")).toBe("many");
      expect(getPluralForm(15, "ru")).toBe("many");
      expect(getPluralForm(20, "ru")).toBe("many");
      expect(getPluralForm(25, "ru")).toBe("many");
    });

    it("handles teen numbers (11-14) as 'many'", () => {
      expect(getPluralForm(11, "ru")).toBe("many");
      expect(getPluralForm(12, "ru")).toBe("many");
      expect(getPluralForm(13, "ru")).toBe("many");
      expect(getPluralForm(14, "ru")).toBe("many");
      expect(getPluralForm(111, "ru")).toBe("many");
      expect(getPluralForm(112, "ru")).toBe("many");
    });
  });
});
