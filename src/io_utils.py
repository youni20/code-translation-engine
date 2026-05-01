def local_file_reader() -> str:
    with open("./inputs/two_sum.cpp", "r") as file:
        file_content = file.read()
        return file_content


def local_file_writer(content: str) -> None:
    with open("./outputs/output.rs", "w") as file:
        file.write(content)
