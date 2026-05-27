import { useState } from "react";
import logo from "@/assets/cashier-svgrepo-com.svg";

interface Props {
  onFinished: () => void;
}

export function SplashScreen({ onFinished }: Props) {
  const [exiting, setExiting] = useState(false);

  const handleAnimationEnd = () => {
    if (!exiting) {
      setExiting(true);
    } else {
      onFinished();
    }
  };

  return (
    <div
      className={`min-h-screen bg-teller-bg flex flex-col items-center justify-center gap-4 ${
        exiting
          ? "animate-out fade-out duration-300 fill-mode-forwards"
          : "animate-in fade-in duration-300"
      }`}
      onAnimationEnd={handleAnimationEnd}
    >
      <img src={logo} alt="Teller" className="w-16 h-16" />
      <span className="text-teller-text text-2xl font-semibold">Teller</span>
    </div>
  );
}
