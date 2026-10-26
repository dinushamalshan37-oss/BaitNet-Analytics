# 🛡️ BaitNet Analytics

> A high-performance OSINT web application built with Rust and Next.js for tracking, analyzing, and visualizing cyber threats, malicious IPs, and scam networks in real-time.

## 📖 Overview

BaitNet Analytics is an Open-Source Intelligence (OSINT) and threat-tracking platform designed to combat online scams. By aggregating data from multiple cybersecurity endpoints, the system analyzes malicious IP addresses, fake domains, and scam networks. Powered by a Rust backend for ultra-fast, concurrent data processing and a Next.js frontend for interactive data visualization, BaitNet Analytics provides actionable intelligence to identify and track cyber threats efficiently.

## ✨ Key Features

* **Real-Time Threat Tracking:** Instantly search and analyze suspicious IP addresses, domains, and crypto wallets.
* **OSINT Data Aggregation:** Fetches and processes data from external APIs (e.g., VirusTotal, Shodan, PhishTank) concurrently.
* **Interactive Dashboard:** Data visualization using maps and charts to display threat origins and risk scores.
* **High-Performance Backend:** Built with Rust and Axum to handle high-concurrency requests and heavy data processing without lag.
* **Secure & OpSec Focused:** Designed with security in mind, providing safe environments for scam-baiting operations and threat analysis.

## 💻 Tech Stack

**Frontend**
* Framework: [Next.js](https://nextjs.org/) (App Router)
* Language: TypeScript
* Styling: Tailwind CSS
* Visualization: Recharts / Leaflet (Maps)

**Backend**
* Language: [Rust](https://www.rust-lang.org/)
* Framework: Axum / Actix-web
* Async Runtime: Tokio

## 🚀 Getting Started

Follow these instructions to set up the project on your local machine for development and testing.

### Prerequisites

* [Node.js](https://nodejs.org/) (v18 or higher)
* [Rust & Cargo](https://rustup.rs/)
* [PostgreSQL](https://www.postgresql.org/)

### Installation

1. **Clone the repository:**
   ```bash
   git clone [https://github.com/dinushamalshan37-oss/BaitNet-Analytics.git](https://github.com/dinushamalshan37-oss/BaitNet-Analytics.git)
   cd BaitNet-Analytics
   BaitNet-Analytics/
├── backend/                # Rust (Axum) API & OSINT Engine
│   ├── src/                # Rust source code (handlers, models, routes)
│   ├── Cargo.toml          # Rust dependencies
│   └── .env                # Backend environment variables
├── frontend/               # Next.js Application
│   ├── app/                # Next.js App Router pages
│   ├── components/         # Reusable React components (UI, Charts, Maps)
│   ├── tailwind.config.js  # Tailwind CSS configuration
│   └── package.json        # Node.js dependencies
└── README.md               # Project documentation
