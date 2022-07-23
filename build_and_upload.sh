set -ex

docker build -t chessatiel .
id=$(docker create chessatiel)
docker cp "${id}:target/release/chessatiel" .
docker rm -v "$id"

gcloud compute scp chessatiel "chessatiel:" --zone europe-west4-a

rm chessatiel

gcloud compute ssh chessatiel --command="sudo systemctl daemon-reload && sudo systemctl enable chessatiel.service && sudo systemctl restart chessatiel.service" --zone europe-west4-a
