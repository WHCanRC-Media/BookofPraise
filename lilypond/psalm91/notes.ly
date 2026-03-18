melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  d2 a'4 a4 g4 e4 f2 g2 a2 r2 \break

  % Line 2
  a2 c4 c4 b4 g4 a2( b2) a2 r2 \break

  % Line 3
  d,2 a'4 a4 g4 e4 f2 g2 a2 r2 \break

  % Line 4
  a2 c4 c4 b4 g4 a2( b2) a2 r2 \break

  % Line 5
  a2 g4 e4 f4 f4 g2 a2 d,2 r2 \break

  % Line 6
  d'2 a4 c4 b4 a4 g2 f2 r2 \break

  % Line 7
  a2 bes4 a4 g4 f4 e4 d4 e2 r2 \break

  % Line 8
  a2 g4 d4 f4 g4 e2 d1 \bar "|."
}
