melody = \relative c' {
  \clef treble
  \key aes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f2 f4( g4 aes4) g2 f2 g2 g4( aes4 bes4) aes4.( g8) f2 \break

  % Line 2
  c'2 bes4( c4 des4) c4( bes8) aes2 bes4.( aes8) g2 f1 \break

  % Line 3
  f2 f4( g4 aes4) g2 f2 g2 g4( aes4 bes4) aes4.( g8) f2 \break

  % Line 4
  c'2 bes4( c4 des4) c4.( bes8) aes2 bes4.( aes8) g2 f1 \break

  % Line 5
  c'2 aes4( bes4 c4) bes2 bes2 aes2 f4( g4 aes4) g2 g2 \break

  % Line 6
  f2 f4( g4 aes4) bes2 bes2 aes2 bes4( aes4 bes4) c1 \break

  % Line 7
  f,2 f4( g4 aes4) g2 f2 g2 g4( aes4 bes4) aes4.( g8) f2 \break

  % Line 8
  c'2 bes4( c4 des4) c4.( bes8) aes2 bes4.( aes8) g2 f1 \bar "|."
}
