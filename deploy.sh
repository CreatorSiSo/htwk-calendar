cd frontend
npm run build

git checkout -b deploy
echo "!dist/" >> .gitignore

git add .
git commit -m "Deploy"
git push -f heroku deploy:main

git checkout main
git branch -D deploy
