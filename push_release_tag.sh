if [ -z "$1" ]; then
    echo "Run with tag name. Note, this prompts a commit and a push!"
    exit 0
fi
NAME="$1"
shift

git tag -f -a "$NAME" && git push origin "$NAME"
