## Introduction
SemanticsDB is a vector embeddings data that uses Actix, a Rust backend framework.
## API endpoints
Below is a list of all available API endpoints and their respective functions:

| HTTP Method | Endpoint                      | Function                                                       |
|-------------|-------------------------------|----------------------------------------------------------------|
| POST        | /project/`{id}`/file            | Upload and link file to project                                |
| POST        | /project/`{id}`/file/embed      | Embed a file for a project                                     |
| DELETE      | /project/`{id}`/file            | Delete a file from the project                                 |
| POST        | /project/`{id}`/similar         | Get k examples of similar text within project                  |
| POST        | /project                      | Create a new project                                           |
| PUT         | /project/`{id}`                 | Update project (name, permissions, permitted users)            |
| DELETE      | /project/`{id}`                 | Delete project                                                 |
| GET         | /project/`{id}`                 | Get a project with list of files by ID                         |
| GET         | /project/`{id}`/file            | Get a file from a project along with download link and boolean |
| GET         | /admin/project/keys           | Get API access keys for a project                               |
| POST        | /admin/project/keys           | Generate a new access key for a given project                  |
| PUT         | /admin/project/keys           | Modify permissions of a given access key                       |
| DELETE      | /admin/project/keys           | Delete access keys for a given project                         |
| GET         | /admin/user/`{id}`              | Get a user by id                                               |
| DELETE      | /admin/user/`{id}`              | Delete a user                                                  |
| POST        | /admin/user                   | Create new user                                                |
| PUT         | /admin/user                   | Update a user                                                  |
| PUT         | /user                         | Update own user info                                           |

## Architecture
The samantics cloud backend features an SQL database, a file store, and an in-memory vector store. The in-memory vector store is shadowed by 
the SQL database (vector embeddings are stored both in the SQL database and in memory). This is because peristance is 
needed (we want to save the state of embeddings if and when the server terminates) and KNN runs significantly faster on in-memory vectors
than vectors housed on the disk. Thus, when the server starts vector embeddings are loaded from the database into RAM. 
### Memory Manager
The memory manager handers vector embeddings stored in RAM. It tracks embeddings attached to each project. Additionally, it handles 
searching for KNN on vector embeddings for a particular project. Upon server initialization, the memory manager searches projects
in the SQL database and loads their respective embeddings into memory. It also handles synchornization between the database and
memory embeddings during runtime. 
### VPSearch
The memory manager employs an algorithm called VPSearch to find the KNN vector embeddings for a text embedding.  VPSearch utilizes a [vantage point tree](https://ieeexplore.ieee.org/document/5202635)
to effeciently search through vector embeddings. 
