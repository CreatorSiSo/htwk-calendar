#!/bin/bash

trap '' INT

push() (
	trap - INT
	echo '--- pushing ---'

	git checkout -b deploy
	echo "!dist/" >> .gitignore

	git add .
	git commit -m "Deploy"
	git push -f heroku deploy:main

	echo '--- finished pushing ---'
)

cd frontend
npm run build && push

echo '--- cleanup ---'
git checkout main
git branch -D deploy
