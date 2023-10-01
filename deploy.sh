cd frontend
npm run build

git checkout -b deploy
echo "!dist/" >> .gitignore

git add .
git commit -m "Deploy"
git push heroku deploy

git checkout main
git branch -D deploy
