locals {
  iap_tag = "iap"
}

data "google_secret_manager_secret_version" "lichess_api_token" {
  secret    = "lichess-api-token"
}

resource "google_compute_instance" "chessatiel" {
  machine_type = "c2d-highcpu-2"
  name         = "chessatiel"

  boot_disk {
    initialize_params {
      image = "debian-cloud/debian-11"
    }
  }

  network_interface {
    network    = google_compute_network.chessatiel.name
    subnetwork = google_compute_network.chessatiel.name
  }

  tags = [local.iap_tag]

  metadata_startup_script = <<EOT
set -ex
echo LICHESS_API_TOKEN=${data.google_secret_manager_secret_version.lichess_api_token.secret_data} >> /etc/environment
cat <<EOF > /etc/systemd/system/chessatiel.service
[Unit]
Description=Chessatiel

[Service]
ExecStart=/home/tim/chessatiel
Environment="LICHESS_API_TOKEN=${data.google_secret_manager_secret_version.lichess_api_token.secret_data}"

[Install]
WantedBy=multi-user.target
EOF
EOT

}
