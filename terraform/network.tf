resource "google_compute_network" "chessatiel" {
  name = "chessatiel"
  auto_create_subnetworks = false
}

resource "google_compute_subnetwork" "chessatiel" {
  ip_cidr_range = "10.0.0.0/24"
  name          = "chessatiel"
  network       = google_compute_network.chessatiel.name
}

resource "google_compute_firewall" "iap" {
  name    = "iap"
  network = google_compute_network.chessatiel.name

  direction = "INGRESS"

  allow {
    protocol = "tcp"
    ports = ["22"]
  }

  source_ranges = ["35.235.240.0/20"]

  target_tags = [local.iap_tag]
}

resource "google_compute_router" "nat" {
  name = "nat"
  network = google_compute_network.chessatiel.name
}

resource "google_compute_router_nat" "nat" {
  name                               = "nat"
  nat_ip_allocate_option             = "AUTO_ONLY"
  router                             = google_compute_router.nat.name
  source_subnetwork_ip_ranges_to_nat = "ALL_SUBNETWORKS_ALL_PRIMARY_IP_RANGES"

  log_config {
    enable = false
    filter = "ERRORS_ONLY"
  }
}
