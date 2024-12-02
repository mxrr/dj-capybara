docker:
	docker build -t kareigu/capybara:latest --build-arg GIT_COMMIT=$(git rev-parse --short HEAD) .
