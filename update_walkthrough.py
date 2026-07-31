import re

with open('/home/zhufan/.gemini/antigravity/brain/2a761348-7e7f-4d33-a058-e498cdda0a00/walkthrough.md', 'r') as f:
    content = f.read()

# Remove everything from "## Deferred Tasks" onwards since they are from the prior session
content = re.sub(r'## Deferred Tasks.*', '', content, flags=re.DOTALL)

with open('/home/zhufan/.gemini/antigravity/brain/2a761348-7e7f-4d33-a058-e498cdda0a00/walkthrough.md', 'w') as f:
    f.write(content)
