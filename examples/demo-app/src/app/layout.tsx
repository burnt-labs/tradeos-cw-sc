"use client";
import "./globals.css";
import { Inter } from "next/font/google";
import { AbstraxionProvider } from "@burnt-labs/abstraxion";

const inter = Inter({ subsets: ["latin"] });

const treasuryContract = process.env.NEXT_PUBLIC_TREASURY_CONTRACT || "";

const treasuryConfig = {
  treasury: treasuryContract,
  gasPrice: "0.025uxion",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}): JSX.Element {
  return (
    <html lang="en">
      <body className={inter.className}>
        <AbstraxionProvider config={treasuryConfig}>
          {children}
        </AbstraxionProvider>
      </body>
    </html>
  );
}

