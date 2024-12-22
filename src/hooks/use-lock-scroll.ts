export function useLockScroll() {
  const lock = () => {
    document.body.style.overflow = "hidden";
  };

  const unlock = () => {
    document.body.style.overflow = "auto";
  };

  return { lock, unlock };
}
