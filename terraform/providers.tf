terraform {
  required_providers {
    google = {
      source = "hashicorp/google"
      version = "4.29.0"
    }
  }
}

provider "google" {
  project = "chessatiel"
  region = "europe-west4"
  zone = "europe-west4-a"
}
