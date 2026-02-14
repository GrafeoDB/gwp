// Package gwp provides a Go client for the GQL Wire Protocol (GWP).
//
// # Quick Start
//
//	conn, err := gwp.Connect(ctx, "localhost:50051")
//	session, err := conn.CreateSession(ctx)
//	cursor, err := session.Execute(ctx, "MATCH (n:Person) RETURN n.name")
//	for {
//	    row, err := cursor.NextRow()
//	    if err != nil { break }
//	    if row == nil { break }
//	    fmt.Println(row)
//	}
//	session.Close(ctx)
//	conn.Close()
package gwp
